// @test-params 15
pub test(value: u64) -> u64 {
    if (value > 10) {
        return value * 2;
    } else {
        return value + 5;
    }
}
