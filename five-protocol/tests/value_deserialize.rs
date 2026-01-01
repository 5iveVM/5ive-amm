use five_protocol::{ProtocolError, ValueRef};

#[test]
fn truncated_buffers_error() {
    fn check(id: u8, len: usize) {
        let mut buf = [0u8; 20]; // Increased to handle U128 (17 bytes) and HeapString/HeapArray (5 bytes)
        buf[0] = id;
        assert_eq!(
            ValueRef::deserialize_from(&buf[..len - 1]),
            Err(ProtocolError::InvalidInstruction)
        );
    }
    let cases = [
        // Protocol types (match types.rs)
        (0u8, 1usize), // Empty: 1 byte
        (1, 2),        // U8: 1 + 1 = 2 bytes
        (4, 9),        // U64: 1 + 8 = 9 bytes (FIXED: was 2)
        (8, 9),        // I64: 1 + 8 = 9 bytes (FIXED: was 3)
        (14, 17),      // U128: 1 + 16 = 17 bytes (FIXED: was 4)
        (9, 2),        // Bool: 1 + 1 = 2 bytes (FIXED: was 5)
        (12, 4),       // AccountRef: 1 + 3 = 4 bytes (FIXED: was 6)
        (10, 3),       // PubkeyRef: 1 + 2 = 3 bytes (FIXED: was 12)
        (13, 2),       // ArrayRef: 1 + 1 = 2 bytes (was 13, unchanged)
        (11, 3),       // StringRef: 1 + 2 = 3 bytes (FIXED: was 14)
        // VM-specific types (15+)
        (15, 3), // InputRef: 1 + 2 = 3 bytes (FIXED: was 7)
        (16, 3), // TempRef: 1 + 2 = 3 bytes (FIXED: was 8)
        (17, 3), // TupleRef: 1 + 2 = 3 bytes (FIXED: was 9)
        (18, 3), // OptionalRef: 1 + 2 = 3 bytes (FIXED: was 10)
        (19, 3), // ResultRef: 1 + 2 = 3 bytes (FIXED: was 11)
        (20, 5), // HeapString: 1 + 4 = 5 bytes (FIXED: was 15)
        (21, 5), // HeapArray: 1 + 4 = 5 bytes (FIXED: was 16)
    ];
    for (id, len) in cases {
        check(id, len);
    }
}
