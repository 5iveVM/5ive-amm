// @test-params 16
pub test(value: u64) -> u64 {
    return (value << 2) + (value >> 2);
}
