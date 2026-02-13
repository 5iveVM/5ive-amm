// @test-params 7
pub test(input: u64) -> u64 {
    let base = input;
    let offset = 10;
    let scaled = base * 2;
    return scaled + offset;
}
