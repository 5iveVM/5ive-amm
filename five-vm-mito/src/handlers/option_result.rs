//! Option and Result operations handler for MitoVM (0xF0-0xFF)
//!
//! This module handles Option and Result types for safe computation.
//! Uses simple AccountRef conventions: 255=None, 254=Err, others=Some/Ok.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
#[cfg(feature = "debug-logs")]
use heapless::String as HString;

// AccountRef convention constants for Option/Result encoding
const ACCOUNT_REF_NONE: u8 = 255; // Option::None
const ACCOUNT_REF_ERR: u8 = 254; // Result::Err
const ACCOUNT_REF_MAX_VALID: u8 = 253; // Max valid account index for Some/Ok

/// Handle Option and Result operations (0xF0-0xFF)
#[inline(never)]
pub fn handle_option_result_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        RESULT_OK => {
            // RESULT_OK - Create Result::Ok value using AccountRef convention
            let value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", value));
                debug_log!("MitoVM: RESULT_OK wrapping value {}", s.as_str());
            }

            // Store value in temp buffer and return AccountRef
            match value {
                ValueRef::AccountRef(account_idx, _offset)
                    if account_idx <= ACCOUNT_REF_MAX_VALID =>
                {
                    // Already a valid Some/Ok reference, return as-is
                    ctx.push(value)?;
                }
                _ => {
                    // Store in temp buffer and create AccountRef
                    let temp_offset = ctx.write_value_to_temp(&value)?;
                    let result_ref = ValueRef::AccountRef(0, temp_offset);
                    ctx.push(result_ref)?;
                }
            }
        }
        RESULT_ERR => {
            // RESULT_ERR - Create Result::Err value using AccountRef convention (254 = Err)
            let error_code = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", error_code));
                debug_log!("MitoVM: RESULT_ERR with error {}", s.as_str());
            }

            let error_u8 = match error_code {
                ValueRef::U8(code) => code,
                ValueRef::U64(code) => code as u8,
                _ => 1, // Default error code
            };

            // Store error code in temp buffer and use account 254 convention
            let temp_offset = ctx.write_value_to_temp(&ValueRef::U8(error_u8))?;
            let result_ref = ValueRef::AccountRef(ACCOUNT_REF_ERR, temp_offset);
            ctx.push(result_ref)?;
        }
        OPTIONAL_SOME => {
            // OPTIONAL_SOME - Create Option::Some value using AccountRef convention
            let value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", value));
                debug_log!("MitoVM: OPTIONAL_SOME wrapping value {}", s.as_str());
            }

            // Always store the value in temp buffer and return AccountRef
            // This ensures proper nesting of Option<Result<T>> and similar composite types
            let temp_offset = ctx.write_value_to_temp(&value)?;
            let option_ref = ValueRef::AccountRef(0, temp_offset);
            ctx.push(option_ref)?;
        }
        OPTIONAL_NONE => {
            // OPTIONAL_NONE - Create Option::None value using AccountRef convention (255 = None)
            debug_log!("MitoVM: OPTIONAL_NONE creating None value");

            let option_ref = ValueRef::AccountRef(ACCOUNT_REF_NONE, 0);
            ctx.push(option_ref)?;
        }
        OPTIONAL_UNWRAP => {
            // OPTIONAL_UNWRAP - Unwrap Optional value (panic if None)
            let optional_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", optional_value));
                debug_log!("MitoVM: OPTIONAL_UNWRAP unwrapping optional {}", s.as_str());
            }

            match optional_value {
                ValueRef::AccountRef(ACCOUNT_REF_NONE, _) => {
                    // None value - panic
                    debug_log!("MitoVM: OPTIONAL_UNWRAP panic - unwrapping None value");
                    return Err(VMErrorCode::InvalidOperation);
                }
                ValueRef::AccountRef(account_idx, offset)
                    if account_idx <= ACCOUNT_REF_MAX_VALID =>
                {
                    // Some value - read from account/temp buffer
                    let value = ctx.read_value_from_temp(offset)?;
                    ctx.push(value)?;
                }
                _ => {
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        OPTIONAL_IS_SOME => {
            // OPTIONAL_IS_SOME - Check if Optional has Some value
            let optional_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", optional_value));
                debug_log!("MitoVM: OPTIONAL_IS_SOME checking optional {}", s.as_str());
            }

            let is_some = match optional_value {
                ValueRef::AccountRef(ACCOUNT_REF_NONE, _) => false, // None
                ValueRef::AccountRef(account_idx, _) if account_idx <= ACCOUNT_REF_MAX_VALID => {
                    true
                } // Some
                _ => false,                                         // Invalid
            };
            ctx.push(ValueRef::Bool(is_some))?;
        }
        OPTIONAL_IS_NONE => {
            // OPTIONAL_IS_NONE - Check if Optional is None
            let optional_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", optional_value));
                debug_log!("MitoVM: OPTIONAL_IS_NONE checking optional {}", s.as_str());
            }

            let is_none = match optional_value {
                ValueRef::AccountRef(ACCOUNT_REF_NONE, _) => true, // None
                ValueRef::AccountRef(account_idx, _) if account_idx <= ACCOUNT_REF_MAX_VALID => {
                    false
                } // Some
                _ => false,                                        // Invalid
            };
            ctx.push(ValueRef::Bool(is_none))?;
        }
        OPTIONAL_GET_VALUE => {
            // OPTIONAL_GET_VALUE - Get value from Optional (unsafe - no None check)
            let optional_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", optional_value));
                debug_log!("MitoVM: OPTIONAL_GET_VALUE extracting from optional {}", s.as_str());
            }

            match optional_value {
                ValueRef::AccountRef(ACCOUNT_REF_NONE, _) => {
                    // Unsafe operation - return empty value for None (undefined behavior)
                    debug_log!("MitoVM: OPTIONAL_GET_VALUE unsafe - extracting from None");
                    ctx.push(ValueRef::Empty)?;
                }
                ValueRef::AccountRef(account_idx, offset)
                    if account_idx <= ACCOUNT_REF_MAX_VALID =>
                {
                    // Some value - read from temp buffer
                    let value = ctx.read_value_from_temp(offset)?;
                    ctx.push(value)?;
                }
                _ => {
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        RESULT_IS_OK => {
            // RESULT_IS_OK - Check if Result is Ok
            let result_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", result_value));
                debug_log!("MitoVM: RESULT_IS_OK checking result {}", s.as_str());
            }

            let is_ok = match result_value {
                ValueRef::AccountRef(ACCOUNT_REF_ERR, _) => false, // Err
                ValueRef::AccountRef(account_idx, _) if account_idx <= ACCOUNT_REF_MAX_VALID => {
                    true
                } // Ok
                _ => false,                                        // Invalid
            };
            ctx.push(ValueRef::Bool(is_ok))?;
        }
        RESULT_IS_ERR => {
            // RESULT_IS_ERR - Check if Result is Err
            let result_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", result_value));
                debug_log!("MitoVM: RESULT_IS_ERR checking result {}", s.as_str());
            }

            let is_err = match result_value {
                ValueRef::AccountRef(ACCOUNT_REF_ERR, _) => true, // Err
                ValueRef::AccountRef(account_idx, _) if account_idx <= ACCOUNT_REF_MAX_VALID => {
                    false
                } // Ok
                _ => false,                                       // Invalid
            };
            ctx.push(ValueRef::Bool(is_err))?;
        }
        RESULT_UNWRAP => {
            // RESULT_UNWRAP - Unwrap Result value (panic if Err)
            let result_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", result_value));
                debug_log!("MitoVM: RESULT_UNWRAP unwrapping result {}", s.as_str());
            }

            match result_value {
                ValueRef::AccountRef(ACCOUNT_REF_ERR, _) => {
                    // Err value - panic
                    debug_log!("MitoVM: RESULT_UNWRAP panic - unwrapping Err value");
                    return Err(VMErrorCode::InvalidOperation);
                }
                ValueRef::AccountRef(account_idx, offset)
                    if account_idx <= ACCOUNT_REF_MAX_VALID =>
                {
                    // Ok value - read from temp buffer
                    let value = ctx.read_value_from_temp(offset)?;
                    ctx.push(value)?;
                }
                _ => {
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        RESULT_GET_VALUE => {
            // RESULT_GET_VALUE - Get Ok value from Result (unsafe - no Err check)
            let result_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", result_value));
                debug_log!("MitoVM: RESULT_GET_VALUE extracting from result {}", s.as_str());
            }

            match result_value {
                ValueRef::AccountRef(ACCOUNT_REF_ERR, _) => {
                    // Unsafe operation - return empty value for Err (undefined behavior)
                    debug_log!("MitoVM: RESULT_GET_VALUE unsafe - extracting from Err");
                    ctx.push(ValueRef::Empty)?;
                }
                ValueRef::AccountRef(account_idx, offset)
                    if account_idx <= ACCOUNT_REF_MAX_VALID =>
                {
                    // Ok value - read from temp buffer
                    let value = ctx.read_value_from_temp(offset)?;
                    ctx.push(value)?;
                }
                _ => {
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        RESULT_GET_ERROR => {
            // RESULT_GET_ERROR - Get error code from Result (unsafe - no Ok check)
            let result_value = ctx.pop()?;
            #[cfg(feature = "debug-logs")]
            {
                let mut s = HString::<128>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", result_value));
                debug_log!("MitoVM: RESULT_GET_ERROR extracting error from result {}", s.as_str());
            }

            match result_value {
                ValueRef::AccountRef(ACCOUNT_REF_ERR, offset) => {
                    // Err value - read error code from temp buffer
                    let error_value = ctx.read_value_from_temp(offset)?;
                    match error_value {
                        ValueRef::U8(code) => ctx.push(ValueRef::U8(code))?,
                        _ => ctx.push(ValueRef::U8(1))?, // Default error
                    }
                }
                ValueRef::AccountRef(account_idx, _) if account_idx <= ACCOUNT_REF_MAX_VALID => {
                    // Unsafe operation - return 0 error code for Ok (undefined behavior)
                    debug_log!("MitoVM: RESULT_GET_ERROR unsafe - extracting error from Ok");
                    ctx.push(ValueRef::U8(0))?;
                }
                _ => {
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        CREATE_TUPLE => {
            debug_log!("MitoVM: CREATE_TUPLE - create tuple");
            let element_count = ctx.fetch_byte()? as usize;

            if (ctx.stack.sp as usize) < element_count {
                return Err(VMErrorCode::StackError);
            }

            // Calculate size
            let mut total_size = 0;
            for i in 0..element_count {
                let idx = ctx.stack.sp as usize - 1 - i;
                let element = ctx.stack.stack[idx];
                total_size += element.serialized_size();
            }

            // Safety check: TupleRef size is u8, so total_size must fit
            if total_size > 255 {
                return Err(VMErrorCode::OutOfMemory);
            }

            let offset = ctx.alloc_temp(total_size as u8)?;

            // Pop elements and write them (reverse order pop, so we write from end to start to preserve order)
            let mut write_pos = offset as usize + total_size;
            for _ in 0..element_count {
                let element = ctx.pop()?;
                let size = element.serialized_size();
                write_pos -= size;
                element.serialize_into(&mut ctx.temp_buffer_mut()[write_pos..write_pos+size])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
            }

            ctx.push(ValueRef::TupleRef(offset, total_size as u8))?;
        }
        TUPLE_GET => {
            debug_log!("MitoVM: TUPLE_GET - get tuple element");
            let index = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
            let tuple_ref = ctx.pop()?;

            let (offset, size) = match tuple_ref {
                ValueRef::TupleRef(o, s) => (o, s),
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            let mut current_offset = offset as usize;
            let end_offset = current_offset + size as usize;
            let mut current_idx = 0;

            loop {
                 if current_offset >= end_offset {
                     return Err(VMErrorCode::IndexOutOfBounds);
                 }
                 let (val, len) = {
                      let temp = ctx.temp_buffer();
                      let v = ValueRef::deserialize_from(&temp[current_offset..])
                          .map_err(|_| VMErrorCode::ProtocolError)?;
                      (v, v.serialized_size())
                 };

                 if current_idx == index {
                     ctx.push(val)?;
                     break;
                 }

                 current_offset += len;
                 current_idx += 1;
            }
        }
        UNPACK_TUPLE => {
            // UNPACK_TUPLE - Unpack tuple elements onto stack
            let tuple_ref = ctx.pop()?;
            match tuple_ref {
                ValueRef::TupleRef(offset, size) => {
                    let mut current_offset = offset as usize;
                    let end_offset = current_offset + size as usize;
                    
                    while current_offset < end_offset {
                        // Scope the immutable borrow
                        let (val, len) = {
                            let temp = ctx.temp_buffer();
                            if current_offset >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            let val = ValueRef::deserialize_from(&temp[current_offset..])
                                .map_err(|_| VMErrorCode::ProtocolError)?;
                            (val, val.serialized_size())
                        };
                        current_offset += len;
                        
                        // Push to stack
                        ctx.push(val)?;
                    }
                }
                _ => {
                    debug_log!("MitoVM: UNPACK_TUPLE expected TupleRef");
                    return Err(VMErrorCode::TypeMismatch);
                }
            }
        }
        _ => {
            debug_log!("MitoVM: Option/Result opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
