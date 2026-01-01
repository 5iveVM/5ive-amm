// @test-params 120 50
pub test(x: u64, y: u64) -> u64 {
    if (x > y) {
        if (x > 100) {
            return x - y;
        } else {
            return x + y;
        }
    } else {
        return y - x;
    }
}