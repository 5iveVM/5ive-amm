//! Variable-Length Encoding (VLE) for Compact Bytecode
//!
//! Implements LEB128-style encoding to reduce bytecode size by encoding
//! `u32` values in 1-5 bytes instead of fixed 4 bytes.

/// Variable-Length Encoding utilities for compact bytecode
pub struct VLE;

impl VLE {
    /// Encode a `u32` using variable-length encoding.
    ///
    /// Encoding format:
    /// - `0..=0x7F`: 1 byte  `[0xxxxxxx]`
    /// - `0x80..=0x3FFF`: 2 bytes `[1xxxxxxx] [0xxxxxxx]`
    /// - `0x4000..=0x1F_FFFF`: 3 bytes `[1xxxxxxx] [1xxxxxxx] [0xxxxxxx]`
    /// - `0x20_0000..=0x0FFF_FFFF`: 4 bytes `[1xxxxxxx] [1xxxxxxx] [1xxxxxxx] [0xxxxxxx]`
    /// - `0x1000_0000..=0xFFFF_FFFF`: 5 bytes `[1xxxxxxx] [1xxxxxxx] [1xxxxxxx] [1xxxxxxx] [0xxxxxxx]`
    #[inline]
    pub const fn encode_u32(value: u32) -> (usize, [u8; 5]) {
        let mut bytes = [0u8; 5];
        let mut remaining = value;
        let mut i = 0;

        while remaining >= 0x80 {
            bytes[i] = ((remaining & 0x7F) as u8) | 0x80;
            remaining >>= 7;
            i += 1;
        }

        bytes[i] = remaining as u8;
        (i + 1, bytes)
    }

    /// Decode a `u32` from variable-length encoding.
    ///
    /// Returns `(decoded_value, bytes_consumed)` where 1-5 bytes may be read.
    #[inline]
    #[inline]
    pub const fn decode_u32(bytes: &[u8]) -> Option<(u32, usize)> {
        if bytes.is_empty() { return None; }
        
        let b0 = bytes[0];
        if b0 & 0x80 == 0 {
            return Some((b0 as u32, 1));
        }
        
        if bytes.len() < 2 { return None; }
        let b1 = bytes[1];
        let r1 = ((b0 & 0x7F) as u32) | ((b1 as u32 & 0x7F) << 7);
        if b1 & 0x80 == 0 {
            return Some((r1, 2));
        }

        if bytes.len() < 3 { return None; }
        let b2 = bytes[2];
        let r2 = r1 | ((b2 as u32 & 0x7F) << 14);
        if b2 & 0x80 == 0 {
            return Some((r2, 3));
        }

        if bytes.len() < 4 { return None; }
        let b3 = bytes[3];
        let r3 = r2 | ((b3 as u32 & 0x7F) << 21);
        if b3 & 0x80 == 0 {
            return Some((r3, 4));
        }

        if bytes.len() < 5 { return None; }
        let b4 = bytes[4];
        // Check for overflow: last byte can only carry 4 bits (28..31)
        if b4 & 0xF0 != 0 { return None; } 
        let r4 = r3 | ((b4 as u32) << 28);
        // Last byte must terminate (implied by 5 byte limit, but v4 format allows explicit termination check if needed, but here we assume 5th byte is end)
        // Standard VLE for u32 implies max 5 bytes. 
        // We should check if b4 indicates continuation (invalid for u32).
        if b4 & 0x80 != 0 { return None; }

        Some((r4, 5))
    }

    /// Calculate the encoded size of a u32 value without encoding it
    #[inline]
    pub const fn encoded_size(value: u32) -> usize {
        if value < 0x80 {
            1
        } else if value < 0x4000 {
            2
        } else if value < 0x20_0000 {
            3
        } else if value < 0x1000_0000 {
            4
        } else {
            5
        }
    }

    /// Encode a u16 value using variable-length encoding
    /// Since u16 max is 65535, we only need 1-2 bytes
    #[inline]
    pub const fn encode_u16(value: u16) -> (usize, [u8; 2]) {
        if value < 128 {
            // 1 byte encoding: [0xxxxxxx]
            (1, [value as u8, 0])
        } else {
            // 2 byte encoding: [1xxxxxxx] [0xxxxxxx]
            let byte1 = ((value & 0x7F) | 0x80) as u8; // Set high bit + low 7 bits
            let byte2 = ((value >> 7) & 0x7F) as u8; // High 9 bits, clear high bit
            (2, [byte1, byte2])
        }
    }

    /// Decode a u16 value from variable-length encoding
    #[inline]
    pub const fn decode_u16(bytes: &[u8]) -> Option<(u16, usize)> {
        if bytes.is_empty() {
            return None;
        }

        let first_byte = bytes[0];

        if first_byte & 0x80 == 0 {
            // 1 byte encoding: [0xxxxxxx]
            Some((first_byte as u16, 1))
        } else if bytes.len() >= 2 {
            // 2 byte encoding: [1xxxxxxx] [0xxxxxxx]
            let second_byte = bytes[1];
            let value = ((first_byte & 0x7F) as u16) | ((second_byte as u16) << 7);
            Some((value, 2))
        } else {
            None // Not enough bytes
        }
    }

    /// Decode a u8 value from variable-length encoding
    #[inline]
    pub const fn decode_u8(bytes: &[u8]) -> Option<(u8, usize)> {
        if bytes.is_empty() {
            return None;
        }
        Some((bytes[0], 1))
    }

    /// Encode a u8 value as a single byte (no continuation bits)
    #[inline]
    pub const fn encode_u8(value: u8) -> (usize, [u8; 1]) {
        (1, [value])
    }

    /// Encode a u64 value using variable-length encoding
    /// Uses 1-10 bytes depending on the value:
    /// - Values 0-127: 1 byte [0xxxxxxx]
    /// - Values 128-16383: 2 bytes [1xxxxxxx] [0xxxxxxx]
    /// - Values 16384-2097151: 3 bytes [1xxxxxxx] [1xxxxxxx] [0xxxxxxx]
    /// - And so on up to 10 bytes for full u64 range
    #[inline]
    pub fn encode_u64(value: u64) -> (usize, [u8; 10]) {
        let mut result = [0u8; 10];
        let mut remaining = value;
        let mut size = 0;

        loop {
            if remaining < 128 {
                // Last byte - no continuation bit
                result[size] = remaining as u8;
                size += 1;
                break;
            } else {
                // More bytes needed - set continuation bit
                result[size] = ((remaining & 0x7F) | 0x80) as u8;
                remaining >>= 7;
                size += 1;
                if size >= 10 {
                    // Should not happen for u64, but safety check
                    break;
                }
            }
        }

        (size, result)
    }

    /// Decode a u64 value from variable-length encoding
    #[inline]
    pub fn decode_u64(bytes: &[u8]) -> Option<(u64, usize)> {
        if bytes.is_empty() {
            return None;
        }

        let mut result = 0u64;
        let mut shift = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            if i >= 10 {
                return None; // Too many bytes
            }

            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                // No continuation bit, we're done
                return Some((result, i + 1));
            }

            shift += 7;
            if shift >= 70 {
                return None; // Would overflow u64
            }
        }

        None // Incomplete encoding
    }

    /// Encode an i64 value using variable-length encoding
    /// Uses zigzag encoding to handle negative numbers efficiently
    #[inline]
    pub fn encode_i64(value: i64) -> (usize, [u8; 10]) {
        // Zigzag encoding: maps signed integers to unsigned
        // -1 -> 1, -2 -> 3, 0 -> 0, 1 -> 2, 2 -> 4, etc.
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        Self::encode_u64(zigzag)
    }

    /// Decode an i64 value from variable-length encoding
    #[inline]
    pub fn decode_i64(bytes: &[u8]) -> Option<(i64, usize)> {
        if let Some((zigzag, consumed)) = Self::decode_u64(bytes) {
            // Reverse zigzag encoding
            let value = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
            Some((value, consumed))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vle_u32_encoding() {
        // Test 1-byte encoding (0-127)
        let (size, bytes) = VLE::encode_u32(42);
        assert_eq!(size, 1);
        assert_eq!(bytes[0], 42);

        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 42);
        assert_eq!(consumed, 1);

        // Test 2-byte encoding (128-16383)
        let (size, bytes) = VLE::encode_u32(1000);
        assert_eq!(size, 2);

        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 1000);
        assert_eq!(consumed, 2);

        // Test 3-byte encoding (16384-0x1F_FFFF)
        let (size, bytes) = VLE::encode_u32(100000);
        assert_eq!(size, 3);

        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 100000);
        assert_eq!(consumed, 3);

        // Boundary just below 0x1F_FFFF
        let (size, bytes) = VLE::encode_u32(0x1F_FFFE);
        assert_eq!(size, 3);
        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 0x1F_FFFE);
        assert_eq!(consumed, 3);

        // Boundary at 0x1F_FFFF
        let (size, bytes) = VLE::encode_u32(0x1F_FFFF);
        assert_eq!(size, 3);
        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 0x1F_FFFF);
        assert_eq!(consumed, 3);

        // Value just above boundary (0x20_0000)
        let (size, bytes) = VLE::encode_u32(0x20_0000);
        assert_eq!(size, 4);
        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, 0x20_0000);
        assert_eq!(consumed, 4);

        // Large value (u32::MAX)
        let (size, bytes) = VLE::encode_u32(u32::MAX);
        assert_eq!(size, 5);
        let (value, consumed) = VLE::decode_u32(&bytes[..size]).unwrap();
        assert_eq!(value, u32::MAX);
        assert_eq!(consumed, 5);

        // Overflowing value should be rejected
        let overflow = [0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        assert!(VLE::decode_u32(&overflow).is_none());
    }

    #[test]
    fn test_vle_u16_encoding() {
        // Test 1-byte encoding
        let (size, bytes) = VLE::encode_u16(100);
        assert_eq!(size, 1);
        assert_eq!(bytes[0], 100);

        let (value, consumed) = VLE::decode_u16(&bytes[..size]).unwrap();
        assert_eq!(value, 100);
        assert_eq!(consumed, 1);

        // Test 2-byte encoding
        let (size, bytes) = VLE::encode_u16(1000);
        assert_eq!(size, 2);

        let (value, consumed) = VLE::decode_u16(&bytes[..size]).unwrap();
        assert_eq!(value, 1000);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_encoded_size() {
        assert_eq!(VLE::encoded_size(42), 1);
        assert_eq!(VLE::encoded_size(127), 1);
        assert_eq!(VLE::encoded_size(128), 2);
        assert_eq!(VLE::encoded_size(16383), 2);
        assert_eq!(VLE::encoded_size(16384), 3);
        assert_eq!(VLE::encoded_size(0x1F_FFFF), 3);
        assert_eq!(VLE::encoded_size(0x20_0000), 4);
        assert_eq!(VLE::encoded_size(0x0FFF_FFFF), 4);
        assert_eq!(VLE::encoded_size(0x1000_0000), 5);
        assert_eq!(VLE::encoded_size(u32::MAX), 5);
    }

    #[test]
    fn test_vle_u64_encoding() {
        // Test small values (1-byte encoding)
        let (size, bytes) = VLE::encode_u64(42);
        assert_eq!(size, 1);
        assert_eq!(bytes[0], 42);

        let (value, consumed) = VLE::decode_u64(&bytes[..size]).unwrap();
        assert_eq!(value, 42);
        assert_eq!(consumed, 1);

        // Test medium values (2-byte encoding)
        let (size, bytes) = VLE::encode_u64(1000);
        assert_eq!(size, 2);

        let (value, consumed) = VLE::decode_u64(&bytes[..size]).unwrap();
        assert_eq!(value, 1000);
        assert_eq!(consumed, 2);

        // Test large values (multiple bytes)
        let (size, bytes) = VLE::encode_u64(u64::MAX);
        assert!(size <= 10); // Should fit in 10 bytes

        let (value, consumed) = VLE::decode_u64(&bytes[..size]).unwrap();
        assert_eq!(value, u64::MAX);
        assert_eq!(consumed, size);
    }

    #[test]
    fn test_vle_i64_encoding() {
        // Test positive values
        let (size, bytes) = VLE::encode_i64(42);
        let (value, consumed) = VLE::decode_i64(&bytes[..size]).unwrap();
        assert_eq!(value, 42);
        assert_eq!(consumed, size);

        // Test negative values
        let (size, bytes) = VLE::encode_i64(-42);
        let (value, consumed) = VLE::decode_i64(&bytes[..size]).unwrap();
        assert_eq!(value, -42);
        assert_eq!(consumed, size);

        // Test zero
        let (size, bytes) = VLE::encode_i64(0);
        let (value, consumed) = VLE::decode_i64(&bytes[..size]).unwrap();
        assert_eq!(value, 0);
        assert_eq!(consumed, size);

        // Test extreme values
        let (size, bytes) = VLE::encode_i64(i64::MAX);
        let (value, consumed) = VLE::decode_i64(&bytes[..size]).unwrap();
        assert_eq!(value, i64::MAX);
        assert_eq!(consumed, size);

        let (size, bytes) = VLE::encode_i64(i64::MIN);
        let (value, consumed) = VLE::decode_i64(&bytes[..size]).unwrap();
        assert_eq!(value, i64::MIN);
        assert_eq!(consumed, size);
    }
}
