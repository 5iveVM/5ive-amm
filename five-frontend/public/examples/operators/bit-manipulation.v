// Bit manipulation pattern: extract byte at position
// @test-params 305419896 1
pub extract_byte(value: u64, position: u64) -> u64 {
    // Shift right by position * 8, then mask to get one byte
    return (value >> (position * 8)) & 255;
}

// Bit manipulation pattern: set bit at position
// @test-params 0 3
pub set_bit(value: u64, position: u64) -> u64 {
    return value | (1 << position);
}

// Bit manipulation pattern: clear bit at position
// @test-params 255 2
pub clear_bit(value: u64, position: u64) -> u64 {
    return value & ~(1 << position);
}
