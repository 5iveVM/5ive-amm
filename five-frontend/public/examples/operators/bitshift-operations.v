// @test-params 16
pub test(value: u64) -> u64 {
    require(value > 0);
    return (value << 2) + (value >> 2);
}
