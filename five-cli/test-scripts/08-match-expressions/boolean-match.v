// @test-params 15
pub test(value: u64) -> u64 {
    match (value > 10) {
        true => {
            return 1;
        }
        false => {
            return 0;
        }
    }
}
