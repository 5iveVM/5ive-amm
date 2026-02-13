// @test-params 20 4
pub test(a: u64, b: u64) -> u64 {
    require(b > 0);
    return (a + b) * 2 - (a / b);
}
