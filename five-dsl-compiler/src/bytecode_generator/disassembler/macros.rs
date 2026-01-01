//! Macros for reducing boilerplate in bytecode decoding and formatting.

/// Decode a PUSH instruction (supports VLE + fixed-width fallbacks).
/// Reduces massive duplication in PUSH_U16/U32/U64 handling.
///
/// Usage: `decode_push_immediate!(offset, bytes, opcode, 16, u16)` for PUSH_U16
macro_rules! decode_push_fixed_width {
    ($offset:expr, $bytes:expr, $width:expr, $type_name:ty) => {{
        let pos = $offset + 1;
        if pos + $width <= $bytes.len() {
            let raw = match $width {
                2 => $bytes[pos..pos + 2]
                    .try_into()
                    .map(|b: [u8; 2]| <$type_name>::from_le_bytes(b) as u64),
                4 => $bytes[pos..pos + 4]
                    .try_into()
                    .map(|b: [u8; 4]| (u32::from_le_bytes(b) as u64)),
                8 => $bytes[pos..pos + 8]
                    .try_into()
                    .map(|b: [u8; 8]| u64::from_le_bytes(b)),
                _ => Err(std::array::TryFromSliceError),
            };
            raw.ok()
        } else {
            None
        }
    }};
}

/// Bounds-check with early return pattern - used 40+ times in original code.
///
/// Usage: `bounds_check!(bytes, offset + 4, "CALL")` returns None if out of bounds
macro_rules! bounds_check {
    ($bytes:expr, $required_pos:expr) => {
        if $required_pos > $bytes.len() {
            return None;
        }
    };
}

/// Extract CALL metadata (either name_ref:N or inline name string).
/// Used in 3 different places - consolidate into macro.
///
/// Returns: (Option<String>, bytes_consumed)
macro_rules! extract_call_metadata {
    ($bytes:expr, $base_offset:expr) => {{
        let mut name_meta = None;
        let mut advance = 4usize; // param_count + addr(2 bytes)

        if $base_offset + advance < $bytes.len() {
            let peek = $bytes[$base_offset + advance];
            if peek == 0xFF {
                if $base_offset + advance + 2 <= $bytes.len() {
                    name_meta = Some(format!("name_ref:{}", $bytes[$base_offset + advance + 1]));
                    advance += 2;
                }
            } else {
                let name_len = peek as usize;
                if $base_offset + advance + 1 + name_len <= $bytes.len() {
                    let start = $base_offset + advance + 1;
                    name_meta = Some(
                        crate::bytecode_generator::disassembler::decoder::read_utf8_string(
                            $bytes, start, name_len,
                        ),
                    );
                    advance += 1 + name_len;
                }
            }
        }
        (name_meta, advance)
    }};
}

/// Extract LOAD_FIELD/STORE_FIELD offset (VLE or fixed-width u32).
/// Used in 4 different places.
///
/// Returns: (Option<u32>, bytes_consumed)
macro_rules! extract_field_offset {
    ($bytes:expr, $after:expr) => {{
        if let Some((v, c)) =
            crate::bytecode_generator::disassembler::decoder::decode_vle_u128(&$bytes[$after..])
        {
            (Some(v as u32), c)
        } else if let Some(raw) =
            crate::bytecode_generator::disassembler::decoder::read_le_u32($bytes, $after)
        {
            (Some(raw), 4)
        } else {
            (None, 0)
        }
    }};
}

/// Helper to safely format bytecode offset in hex notation (0xABCD or 00AB).
macro_rules! format_offset_hex {
    ($offset:expr) => {
        format!("{:04X}", $offset)
    };
}

/// Safe bounds check that returns an option for inline usage.
/// Returns None if check fails, Some(()) otherwise.
macro_rules! check_bounds {
    ($bytes:expr, $required_end:expr) => {
        if $required_end <= $bytes.len() {
            Some(())
        } else {
            None
        }
    };
}
