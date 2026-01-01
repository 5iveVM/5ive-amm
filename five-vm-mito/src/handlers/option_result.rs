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
        _ => {
            debug_log!("MitoVM: Option/Result opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
