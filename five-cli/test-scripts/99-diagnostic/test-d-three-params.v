// @test-params 10 20 30
// Test D: Three parameters (test multi-param varint encoding)
pub three_params(a: u64, b: u64, c: u64) -> u64 {
    return a + b + c;
}
