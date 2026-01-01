// @test-params 50 30
pub test(x: u64, y: u64) -> bool {
    return (x > y) && (x < 100) || (y == 0);
}
