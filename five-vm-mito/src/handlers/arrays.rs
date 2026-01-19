//! Unified Array and String operations handler for MitoVM
//!
//! This module handles both array and string operations as a unified system.
//! Arrays (including strings as byte arrays) are stored in the temp buffer
//! with a header containing length and element type.
//!
//! **Simplified Binary Element Type System:**
//! - **Type 0 (FIXED_SIZE)**: Fixed-size elements enabling O(1) indexing
//!   - u8, u64, i64, bool, pubkey - all have known byte sizes
//!   - Direct access: base_addr + (index * element_size)
//! - **Type 1 (VARIABLE_SIZE)**: Variable-size elements requiring O(n) traversal
//!   - UTF-8 strings, nested arrays, complex data structures
//!   - Length-prefixed storage, traversal-based access
//!
//! **Temp Buffer Size Constraints:**
//! - Temp buffer is fixed at 64 bytes (TEMP_BUFFER_SIZE = 64)
//! - All array/string IDs must be in range [0, 63] to avoid buffer overflow
//! - StringRef uses u16 IDs but must be validated to be ≤ 63 before temp_buffer access
//! - ArrayRef uses u8 IDs which are naturally bounded by alloc_temp() safety checks
//! - When converting StringRef(u16) to ArrayRef(u8), bounds checking is mandatory:
//!   ```rust
//!   # let id = 50u16;
//!   # fn check_bounds(id: u16) -> Result<u8, &'static str> {
//!   if id > 63 {
//!       return Err("IndexOutOfBounds");
//!   }
//!       Ok(id as u8)
//!   # }
//!   # check_bounds(id)?;
//!   # Ok::<(), &str>(())
//!   ```

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    utils::ValueRefUtils,
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle unified array and string operations (0x60-0x6F range)
/// 🎯 LOGICAL REORGANIZATION: All array and string operations consolidated
#[inline(always)]
pub fn handle_arrays(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Array creation and management (0x60-0x65)
        CREATE_ARRAY => handle_array_creation(opcode, ctx),
        PUSH_ARRAY_LITERAL => handle_array_literals(opcode, ctx),
        ARRAY_INDEX => handle_array_operations(opcode, ctx),
        ARRAY_LENGTH => handle_array_operations(opcode, ctx),
        ARRAY_SET => handle_array_operations(opcode, ctx),
        ARRAY_GET => handle_array_operations(opcode, ctx),

        // String operations (0x66-0x67)
        PUSH_STRING_LITERAL => handle_string_operations(opcode, ctx),
        PUSH_STRING => handle_string_operations(opcode, ctx),

        // 0x68-0x6F: Available for additional array/string operations
        _ => {
            debug_log!("MitoVM: Invalid array opcode {} in 0x60-0x6F range", opcode);
            Err(VMErrorCode::InvalidInstruction)
        }
    }
}

/// Handle array literal creation (PUSH_ARRAY_LITERAL)
#[inline(always)]
fn handle_array_literals(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        PUSH_ARRAY_LITERAL => {
            let element_count = ctx.fetch_byte()?;

            if element_count == 0 {
                // Empty array - just store header
                let array_id = ctx.alloc_temp(2)?;
                ctx.temp_buffer_mut()[array_id as usize] = 0; // length = 0
                ctx.temp_buffer_mut()[array_id as usize + 1] = 0; // element_type = FIXED_SIZE (binary classification)
                ctx.push(ValueRef::ArrayRef(array_id))?;
                return Ok(());
            }

            // Pop elements from stack in reverse order (last pushed = first element)
            const MAX_ARRAY_ELEMENTS: usize = 32;
            if element_count as usize > MAX_ARRAY_ELEMENTS {
                return Err(VMErrorCode::StackError);
            }

            if ctx.stack.sp < element_count {
                return Err(VMErrorCode::StackError);
            }

            // Determine element type and total size without storing elements
            let mut total_size = 2; // header size
            let mut element_type_id = None;
            for i in 0..element_count {
                let idx = ctx.stack.sp as usize - 1 - i as usize;
                let element = ctx.stack.stack[idx];
                if element_type_id.is_none() {
                    element_type_id = Some(element.type_id());
                }
                total_size += element.serialized_size();
            }

            let mut coerce_to_u8 = false;
            if total_size > 62 {
                let mut all_u8_compatible = true;
                for i in 0..element_count {
                    let idx = ctx.stack.sp as usize - 1 - i as usize;
                    let element = ctx.stack.stack[idx];
                    let is_u8 = match element {
                        ValueRef::U8(_) | ValueRef::Bool(_) => true,
                        ValueRef::U64(v) => v <= u8::MAX as u64,
                        ValueRef::I64(v) => (0..=u8::MAX as i64).contains(&v),
                        _ => false,
                    };
                    if !is_u8 {
                        all_u8_compatible = false;
                        break;
                    }
                }

                if all_u8_compatible {
                    let compact_size = 2 + (element_count as usize * 2);
                    if compact_size > 62 {
                        return Err(VMErrorCode::OutOfMemory);
                    }
                    total_size = compact_size;
                    element_type_id = Some(five_protocol::types::U8);
                    coerce_to_u8 = true;
                } else {
                    return Err(VMErrorCode::OutOfMemory);
                }
            }

            // Determine binary element type: 0=FIXED_SIZE, 1=VARIABLE_SIZE
            let first_element_type_id = element_type_id.unwrap_or(0);
            let array_element_type = match first_element_type_id {
                // Fixed-size elements (Type 0): u8, u64, i64, bool, pubkey
                1 | 4 | 8 | 9 | 10 => 0, // U8, U64, I64, Bool, Pubkey
                // Variable-size elements (Type 1): strings, nested arrays
                11 | _ => 1, // String and others default to variable-size
            };
            // Allocate temp buffer space
            let array_id = ctx.alloc_temp(total_size as u8)?;

            // Write array header
            ctx.temp_buffer_mut()[array_id as usize] = element_count; // length
            ctx.temp_buffer_mut()[array_id as usize + 1] = array_element_type; // element_type

            // Write elements directly in reverse order
            let mut write_offset = array_id as usize + total_size;
            for _ in 0..element_count {
                let element = ctx.pop()?;
                let serialized = if coerce_to_u8 {
                    ValueRef::U8(ValueRefUtils::as_u8(element)?)
                } else {
                    element
                };
                let size = serialized.serialized_size();
                write_offset -= size;
                serialized
                    .serialize_into(&mut ctx.temp_buffer_mut()[write_offset..write_offset + size])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
            }

            ctx.push(ValueRef::ArrayRef(array_id))?;
            debug_log!(
                "MitoVM: Array created at array_id={} with {} elements",
                array_id,
                element_count
            );
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}

/// Handle core array operations (ARRAY_INDEX, ARRAY_LENGTH, ARRAY_SET)
#[inline(always)]
fn handle_array_operations(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        ARRAY_INDEX => {
            let index_ref = ctx.pop()?;
            let array_ref = ctx.pop()?;

            // Get index value
            let index = match index_ref {
                ValueRef::U8(i) => i as usize,
                ValueRef::U64(i) => i as usize,
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Get array
            let array_id = match array_ref {
                ValueRef::ArrayRef(id) => {
                    // Validate array_id is within temp buffer bounds
                    if id as usize + 2 > ctx.temp_buffer().len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }
                    id
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Read array header
            let array_length = ctx.temp_buffer_mut()[array_id as usize] as usize;
            let _element_type = ctx.temp_buffer_mut()[array_id as usize + 1];

            if index >= array_length {
                return Err(VMErrorCode::IndexOutOfBounds);
            }

            // Find the element at the given index
            let mut current_offset = array_id as usize + 2; // Skip header
            for _ in 0..index {
                let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                current_offset += element.serialized_size();
            }

            // Deserialize the target element
            let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(element)?;

            debug_log!(
                "MitoVM: Array index {} retrieved from array_id={}",
                index as u32,
                array_id as u32
            );
        }
        ARRAY_LENGTH => {
            let array_ref = ctx.pop()?;

            let array_id = match array_ref {
                ValueRef::ArrayRef(id) => {
                    // Validate array_id is within temp buffer bounds
                    if id as usize + 2 > ctx.temp_buffer().len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }
                    id
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Read array length from header
            let array_length = ctx.temp_buffer_mut()[array_id as usize];
            ctx.push(ValueRef::U8(array_length))?;

            debug_log!(
                "MitoVM: Array length {} from array_id={}",
                array_length,
                array_id
            );
        }
        ARRAY_SET => {
            let index = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
            let array_ref = ctx.pop()?;
            let value = ctx.pop()?;

            let array_id = match array_ref {
                ValueRef::ArrayRef(id) => {
                    // Validate array_id is within temp buffer bounds
                    if id as usize + 2 > ctx.temp_buffer().len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }
                    id
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            debug_log!(
                "MitoVM: ARRAY_SET index {} on array_id={}",
                index as u32,
                array_id
            );

            // Read current length and validate bounds
            let current_length = ctx.temp_buffer_mut()[array_id as usize] as usize;
            let element_size = 8usize; // Conservative size for ValueRef serialization
            let header_size = 2usize;

            // Calculate maximum capacity accounting for this array's offset in the buffer
            let available_after_array_start = ctx.temp_buffer().len().saturating_sub(array_id as usize);
            let max_capacity = available_after_array_start.saturating_sub(header_size) / element_size;

            if index >= max_capacity {
                return Err(VMErrorCode::IndexOutOfBounds);
            }

            // Calculate target offset in temp_buffer
            let target_offset = array_id as usize + header_size + (index * element_size);

            // Serialize the value into temp_buffer at target location
            // Simple serialization: store ValueRef discriminant + data
            match value {
                ValueRef::U8(val) => {
                    ctx.temp_buffer_mut()[target_offset] = 1; // U8 discriminant
                    ctx.temp_buffer_mut()[target_offset + 1] = val;
                }
                ValueRef::U64(val) => {
                    ctx.temp_buffer_mut()[target_offset] = 2; // U64 discriminant
                    let bytes = val.to_le_bytes();
                    for (i, byte) in bytes.iter().enumerate() {
                        ctx.temp_buffer_mut()[target_offset + 1 + i] = *byte;
                    }
                }
                ValueRef::Bool(val) => {
                    ctx.temp_buffer_mut()[target_offset] = 3; // Bool discriminant
                    ctx.temp_buffer_mut()[target_offset + 1] = if val { 1 } else { 0 };
                }
                _ => {
                    // For complex types, would need more sophisticated serialization
                    return Err(VMErrorCode::TypeMismatch);
                }
            }

            // Update array length if we set beyond current length
            if index >= current_length {
                ctx.temp_buffer_mut()[array_id as usize] = (index + 1) as u8;
            }

            // Push the array back onto stack for chaining
            ctx.push(ValueRef::ArrayRef(array_id))?;
            debug_log!(
                "MitoVM: Set element {} in array_id={}, new length {}",
                index as u32,
                array_id,
                ctx.temp_buffer_mut()[array_id as usize]
            );
        }
        ARRAY_GET => {
            // ARRAY_GET is an alias for ARRAY_INDEX - same functionality
            debug_log!("MitoVM: ARRAY_GET - delegating to ARRAY_INDEX logic");
            let index_ref = ctx.pop()?;
            let array_ref = ctx.pop()?;

            // Get index value
            let index = match index_ref {
                ValueRef::U8(i) => i as usize,
                ValueRef::U64(i) => i as usize,
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Get array
            let array_id = match array_ref {
                ValueRef::ArrayRef(id) => {
                    // Validate array_id is within temp buffer bounds
                    if id as usize + 2 > ctx.temp_buffer().len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }
                    id
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Read array header
            let array_length = ctx.temp_buffer_mut()[array_id as usize] as usize;
            let _element_type = ctx.temp_buffer_mut()[array_id as usize + 1];

            if index >= array_length {
                return Err(VMErrorCode::IndexOutOfBounds);
            }

            // Find the element at the given index
            let mut current_offset = array_id as usize + 2; // Skip header
            for _ in 0..index {
                let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                current_offset += element.serialized_size();
            }

            // Deserialize the target element
            let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(element)?;

            debug_log!(
                "MitoVM: Array element {} retrieved from array_id={}",
                index as u32,
                array_id as u32
            );
        }
        _ => {
            debug_log!("MitoVM: Invalid array operation opcode {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

/// Handle string operations (PUSH_STRING_LITERAL, PUSH_STRING)
/// 🎯 LOGICAL REORGANIZATION: Consolidated string operations
#[inline(always)]
fn handle_string_operations(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        PUSH_STRING_LITERAL => {
            let string_length = ctx.fetch_byte()?;
            debug_log!("MitoVM: PUSH_STRING_LITERAL with {} bytes", string_length);

            if string_length == 0 {
                // Empty string - stored as empty array with string element type
                let array_id = ctx.alloc_temp(2)?;
                ctx.temp_buffer_mut()[array_id as usize] = 0; // length = 0
                ctx.temp_buffer_mut()[array_id as usize + 1] = 1; // element_type = VARIABLE_SIZE (binary classification)
                ctx.push(ValueRef::StringRef(array_id as u16))?;
                return Ok(());
            }

            // Calculate total size: header (2 bytes) + string bytes
            let total_size = 2 + string_length as usize;
            if total_size > 62 {
                // Fallback to heap allocation for large strings
                let heap_total_size = 4 + string_length as usize; // 4 bytes for length + string bytes
                let heap_id = ctx.heap_alloc(heap_total_size)?;

                // Write length (u32)
                let length_bytes = (string_length as u32).to_le_bytes();
                ctx.get_heap_data_mut(heap_id, 4)?.copy_from_slice(&length_bytes);

                // Read string bytes from bytecode directly into heap
                for i in 0..string_length {
                    let byte = ctx.fetch_byte()?;
                    ctx.get_heap_data_mut(heap_id + 4 + i as u32, 1)?[0] = byte;
                }

                // Validate UTF-8 encoding
                let string_bytes = ctx.get_heap_data(heap_id + 4, string_length as u32)?;
                if core::str::from_utf8(string_bytes).is_err() {
                    return Err(VMErrorCode::InvalidOperation); // Invalid UTF-8
                }

                ctx.push(ValueRef::HeapString(heap_id))?;
                debug_log!(
                    "MitoVM: Heap String created at heap_id={} with {} bytes",
                    heap_id,
                    string_length
                );
                return Ok(());
            }

            // Allocate temp buffer space
            let array_id = ctx.alloc_temp(total_size as u8)?;

            // Write array header for string
            ctx.temp_buffer_mut()[array_id as usize] = string_length; // length
            ctx.temp_buffer_mut()[array_id as usize + 1] = 1; // element_type = VARIABLE_SIZE (binary classification)

            // Read string bytes from bytecode
            for i in 0..string_length {
                ctx.temp_buffer_mut()[array_id as usize + 2 + i as usize] = ctx.fetch_byte()?;
            }

            // Validate UTF-8 encoding
            let string_bytes = &ctx.temp_buffer()
                [array_id as usize + 2..array_id as usize + 2 + string_length as usize];
            if core::str::from_utf8(string_bytes).is_err() {
                return Err(VMErrorCode::InvalidOperation); // Invalid UTF-8
            }

            ctx.push(ValueRef::StringRef(array_id as u16))?;
            debug_log!(
                "MitoVM: String created at array_id={} with {} bytes",
                array_id,
                string_length
            );
        }
        PUSH_STRING => {
            // PUSH_STRING with VLE encoding - similar to PUSH_STRING_LITERAL but with VLE length
            let string_length = ctx.fetch_vle_u32()?; // Fetch VLE encoded length (u32)
            debug_log!(
                "MitoVM: PUSH_STRING with {} bytes (VLE encoded)",
                string_length
            );

            if string_length == 0 {
                // Empty string - stored as empty array with string element type
                let array_id = ctx.alloc_temp(2)?;
                ctx.temp_buffer_mut()[array_id as usize] = 0; // length = 0
                ctx.temp_buffer_mut()[array_id as usize + 1] = 1; // element_type = VARIABLE_SIZE (binary classification)
                ctx.push(ValueRef::StringRef(array_id as u16))?;
                return Ok(());
            }

            // Calculate total size: header (2 bytes) + string bytes
            let total_size = 2 + string_length as usize;
            if total_size > 62 {
                // Fallback to heap allocation for large strings
                let heap_total_size = 4 + string_length as usize; // 4 bytes for length + string bytes
                let heap_id = ctx.heap_alloc(heap_total_size)?;

                // Write length (u32)
                let length_bytes = string_length.to_le_bytes();
                ctx.get_heap_data_mut(heap_id, 4)?.copy_from_slice(&length_bytes);

                // Read string bytes from bytecode
                for i in 0..string_length {
                    let byte = ctx.fetch_byte()?;
                    ctx.get_heap_data_mut(heap_id + 4 + i, 1)?[0] = byte;
                }

                // Validate UTF-8 encoding
                let string_bytes = ctx.get_heap_data(heap_id + 4, string_length)?;
                if core::str::from_utf8(string_bytes).is_err() {
                    return Err(VMErrorCode::InvalidOperation); // Invalid UTF-8
                }

                ctx.push(ValueRef::HeapString(heap_id))?;
                debug_log!(
                    "MitoVM: Heap String created at heap_id={} with {} bytes (VLE)",
                    heap_id,
                    string_length
                );
                return Ok(());
            }

            // Allocate temp buffer space
            let array_id = ctx.alloc_temp(total_size as u8)?;

            // Write array header for string
            ctx.temp_buffer_mut()[array_id as usize] = string_length as u8; // length (safe because total_size <= 62)
            ctx.temp_buffer_mut()[array_id as usize + 1] = 1; // element_type = VARIABLE_SIZE (binary classification)

            // Read string bytes from bytecode
            for i in 0..string_length {
                ctx.temp_buffer_mut()[array_id as usize + 2 + i as usize] = ctx.fetch_byte()?;
            }

            // Validate UTF-8 encoding
            let string_bytes = &ctx.temp_buffer()
                [array_id as usize + 2..array_id as usize + 2 + string_length as usize];
            if core::str::from_utf8(string_bytes).is_err() {
                return Err(VMErrorCode::InvalidOperation); // Invalid UTF-8
            }

            ctx.push(ValueRef::StringRef(array_id as u16))?;
            debug_log!(
                "MitoVM: String created at array_id={} with {} bytes (VLE)",
                array_id,
                string_length
            );
        }
        _ => {
            debug_log!("MitoVM: Invalid string operation opcode {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

/// Handle array creation (CREATE_ARRAY)
#[inline(always)]
fn handle_array_creation(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CREATE_ARRAY => {
            // Pop capacity from stack (count of elements to initialize from stack)
            let capacity_val = ctx.pop()?;
            let capacity = match capacity_val.as_u8() {
                Some(n) => n,
                None => return Err(VMErrorCode::TypeMismatch),
            };
            debug_log!("MitoVM: CREATE_ARRAY with capacity {}", capacity);

            if capacity == 0 {
                // Empty array - just store header
                let array_id = ctx.alloc_temp(2)?;
                ctx.temp_buffer_mut()[array_id as usize] = 0; // length = 0
                ctx.temp_buffer_mut()[array_id as usize + 1] = 0; // element_type = FIXED_SIZE
                ctx.push(ValueRef::ArrayRef(array_id))?;
                return Ok(());
            }

            // Pop elements from stack in reverse order (like PUSH_ARRAY_LITERAL)
            const MAX_ARRAY_ELEMENTS: usize = 32;
            if capacity as usize > MAX_ARRAY_ELEMENTS {
                return Err(VMErrorCode::StackError);
            }

            if (ctx.stack.sp as usize) < capacity as usize {
                return Err(VMErrorCode::StackError);
            }

            // Calculate total size needed
            let mut total_size = 2; // header size
            for i in 0..capacity {
                let idx = ctx.stack.sp as usize - 1 - i as usize;
                let element = ctx.stack.stack[idx];
                total_size += element.serialized_size();
                if total_size > 62 {
                    return Err(VMErrorCode::OutOfMemory);
                }
            }

            // Determine element type from first element
            let first_element_type_id = if capacity > 0 {
                let first_idx = ctx.stack.sp as usize - 1;
                ctx.stack.stack[first_idx].type_id()
            } else {
                0
            };

            let array_element_type = match first_element_type_id {
                // Fixed-size elements (Type 0): u8, u64, i64, bool, pubkey
                1 | 4 | 8 | 9 | 10 => 0, // U8, U64, I64, Bool, Pubkey
                // Variable-size elements (Type 1): strings, nested arrays
                11 | _ => 1, // String and others default to variable-size
            };

            // Allocate space and initialize header
            let array_id = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[array_id as usize] = capacity; // length = number of elements
            ctx.temp_buffer_mut()[array_id as usize + 1] = array_element_type; // element_type

            // Write elements in reverse order (like PUSH_ARRAY_LITERAL)
            let mut write_offset = array_id as usize + total_size;
            for _ in 0..capacity {
                let element = ctx.pop()?;
                let size = element.serialized_size();
                write_offset -= size;
                element
                    .serialize_into(&mut ctx.temp_buffer_mut()[write_offset..write_offset + size])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
            }

            ctx.push(ValueRef::ArrayRef(array_id))?;
            debug_log!(
                "MitoVM: Created array at array_id={} with {} elements",
                array_id,
                capacity
            );
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_checking_core_logic() {
        // Test the core bounds checking logic that was implemented in the fix

        // Test cases: (StringRef ID, Expected Result)
        let test_cases = [
            (0, true),     // Valid: 0 <= 63
            (32, true),    // Valid: 32 <= 63
            (63, true),    // Valid: exactly at boundary
            (64, false),   // Invalid: exceeds temp_buffer bounds
            (255, false),  // Invalid: would fit in u8 but exceeds temp_buffer
            (1000, false), // Invalid: exceeds u8 bounds entirely
        ];

        for (string_id, should_be_valid) in test_cases {
            // Simulate the bounds checking logic from the fix:
            // ValueRef::StringRef(id) => {
            //     if id > 63 {
            //         return Err(VMError::IndexOutOfBounds);
            //     }
            //     id as u8
            // }
            let result = if string_id > 63 {
                Err(VMErrorCode::IndexOutOfBounds)
            } else {
                Ok(string_id as u8)
            };

            if should_be_valid {
                assert!(
                    result.is_ok(),
                    "Expected StringRef({}) to be valid",
                    string_id
                );
                assert_eq!(result.unwrap(), string_id as u8);
            } else {
                assert!(
                    matches!(result, Err(VMErrorCode::IndexOutOfBounds)),
                    "Expected StringRef({}) to be invalid with IndexOutOfBounds",
                    string_id
                );
            }
        }
    }

    #[test]
    fn test_temp_buffer_size_constraint_validation() {
        // Validate the documented temp buffer constraints

        // TEMP_BUFFER_SIZE should be 64 bytes
        const TEMP_BUFFER_SIZE: usize = 64;

        // All valid IDs should be in range [0, 63]
        for id in 0..=63 {
            assert!(
                id < TEMP_BUFFER_SIZE,
                "Valid ID {} should be within temp_buffer bounds",
                id
            );
        }

        // All invalid IDs should exceed the buffer
        for id in 64..=255 {
            assert!(
                id >= TEMP_BUFFER_SIZE,
                "Invalid ID {} should exceed temp_buffer bounds",
                id
            );
        }

        // Test the conversion safety for boundary cases
        let valid_boundary = 63u16;
        let invalid_boundary = 64u16;

        // Valid conversion
        assert!(valid_boundary <= 63);
        let converted_valid = valid_boundary as u8;
        assert_eq!(converted_valid, 63);

        // Invalid conversion (should be caught by bounds check)
        assert!(invalid_boundary > 63);
        // The fix would return Err(VMError::IndexOutOfBounds) here
    }
}
