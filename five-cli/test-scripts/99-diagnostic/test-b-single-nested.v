// @test-params 5
// Test B: Single nested call with parameter passing (likely FAIL on-chain)
pub outer(x: u64) -> u64 {
    return helper(x + 10);
}

pub helper(y: u64) -> u64 {
    return y * 2;
}
