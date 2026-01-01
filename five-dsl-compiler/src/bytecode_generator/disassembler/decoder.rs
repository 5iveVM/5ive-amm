//! Low-level bytecode decoding utilities for VLE and byte extraction.

use std::convert::TryInto;

/// Decode a VLE-encoded unsigned integer from the beginning of `data`.
/// Returns Some((value, bytes_consumed)) on success, or None if truncated.
pub fn decode_vle_u128(data: &[u8]) -> Option<(u128, usize)> {
    let mut acc: u128 = 0;
    let mut shift = 0usize;
    let mut consumed = 0usize;
    for &byte in data.iter() {
        let low = (byte & 0x7F) as u128;
        acc |= low << shift;
        consumed += 1;
        if (byte & 0x80) == 0 {
            return Some((acc, consumed));
        }
        shift += 7;
        if shift >= 128 {
            return None;
        }
    }
    None
}

/// Safely extract a u16 from bytes at position, or return None if bounds exceeded.
pub fn read_le_u16(bytes: &[u8], pos: usize) -> Option<u16> {
    if pos + 2 <= bytes.len() {
        Some(u16::from_le_bytes(bytes[pos..pos + 2].try_into().unwrap()))
    } else {
        None
    }
}

/// Safely extract a u32 from bytes at position, or return None if bounds exceeded.
pub fn read_le_u32(bytes: &[u8], pos: usize) -> Option<u32> {
    if pos + 4 <= bytes.len() {
        Some(u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()))
    } else {
        None
    }
}

/// Safely extract a u64 from bytes at position, or return None if bounds exceeded.
pub fn read_le_u64(bytes: &[u8], pos: usize) -> Option<u64> {
    if pos + 8 <= bytes.len() {
        Some(u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap()))
    } else {
        None
    }
}

/// Safely extract a byte at position, or return None if bounds exceeded.
pub fn read_byte(bytes: &[u8], pos: usize) -> Option<u8> {
    if pos < bytes.len() {
        Some(bytes[pos])
    } else {
        None
    }
}

/// Try to read UTF-8 string of given length, or return error string.
pub fn read_utf8_string(bytes: &[u8], start: usize, len: usize) -> String {
    if start + len <= bytes.len() {
        match std::str::from_utf8(&bytes[start..start + len]) {
            Ok(s) => s.to_string(),
            Err(_) => "<non-utf8-name>".to_string(),
        }
    } else {
        "<invalid-bounds>".to_string()
    }
}
