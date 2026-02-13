// @test-params 255 15
pub test(a: u64, b: u64) -> u64 {
    require(a > 0 || b > 0);
    return (a & b) + (a | b);
}
