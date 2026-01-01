//! Shared CALL instruction decoding to eliminate duplication.

use crate::bytecode_generator::disassembler::decoder::*;
use crate::bytecode_generator::disassembler::types::CallSite;

/// Decode a single CALL instruction at the given offset.
/// Handles parameter extraction, function address, and optional name metadata.
pub fn decode_call_at(bytes: &[u8], offset: usize) -> Option<CallSite> {
    if offset + 4 > bytes.len() {
        return None;
    }

    let param_count = bytes[offset + 1];
    let addr_lo = bytes[offset + 2];
    let addr_hi = bytes[offset + 3];
    let function_address = u16::from_le_bytes([addr_lo, addr_hi]);

    let mut name_meta = None;
    let base_adv = 4usize;

    if offset + base_adv < bytes.len() {
        let peek = bytes[offset + base_adv];
        if peek == 0xFF {
            if offset + base_adv + 2 <= bytes.len() {
                name_meta = Some(format!("name_ref:{}", bytes[offset + base_adv + 1]));
            }
        } else {
            let name_len = peek as usize;
            if offset + base_adv + 1 + name_len <= bytes.len() {
                let start = offset + base_adv + 1;
                name_meta = Some(read_utf8_string(bytes, start, name_len));
            }
        }
    }

    Some(CallSite {
        offset,
        param_count,
        function_address,
        name_metadata: name_meta,
    })
}

/// Compute how many bytes a CALL instruction consumes (including metadata).
pub fn call_size(bytes: &[u8], offset: usize) -> usize {
    if offset + 4 > bytes.len() {
        return 0;
    }

    let mut size = 4usize;
    if offset + size < bytes.len() {
        let peek = bytes[offset + size];
        if peek == 0xFF {
            if offset + size + 2 <= bytes.len() {
                size += 2;
            }
        } else {
            let name_len = peek as usize;
            if offset + size + 1 + name_len <= bytes.len() {
                size += 1 + name_len;
            }
        }
    }
    size
}
