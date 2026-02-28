//! Shared CALL instruction decoding to eliminate duplication.

use crate::bytecode_generator::disassembler::types::CallSite;

/// Decode a single CALL instruction at the given offset.
/// Handles parameter extraction and function address.
pub fn decode_call_at(bytes: &[u8], offset: usize) -> Option<CallSite> {
    if offset + 4 > bytes.len() {
        return None;
    }

    let param_count = bytes[offset + 1];
    let addr_lo = bytes[offset + 2];
    let addr_hi = bytes[offset + 3];
    let function_address = u16::from_le_bytes([addr_lo, addr_hi]);

    Some(CallSite {
        offset,
        param_count,
        function_address,
        name_metadata: None,
    })
}

/// Compute how many bytes a CALL instruction consumes.
/// CALL is fixed-width: opcode(1) + param_count(1) + function_address(2).
pub fn call_size(bytes: &[u8], offset: usize) -> usize {
    if offset + 4 > bytes.len() {
        0
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_protocol::opcodes;

    #[test]
    fn call_size_is_fixed_width_even_with_metadata_like_bytes() {
        let bytes = vec![opcodes::CALL, 2, 0x34, 0x12, 0xFF, 0x3F, opcodes::HALT];
        assert_eq!(call_size(&bytes, 0), 4);
    }

    #[test]
    fn decode_call_ignores_following_bytes_as_metadata() {
        let bytes = vec![opcodes::CALL, 3, 0x09, 0x00, 0xAA, 0xBB];
        let call = decode_call_at(&bytes, 0).expect("decode");
        assert_eq!(call.param_count, 3);
        assert_eq!(call.function_address, 9);
        assert!(call.name_metadata.is_none());
    }
}
