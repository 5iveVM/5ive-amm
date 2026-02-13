// @test-params 5
pub test(limit: u64) -> u64 {
    require(limit < 100);
    let count = 0;
    while (count < limit) {
        count = count + 1;
    }
    return count;
}
